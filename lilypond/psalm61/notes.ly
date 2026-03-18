melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 e2 a4 a4 g4 c2 b4 a2 g4 \break

  % Line 2
  c2 b4 a2 g2 r2 \break

  % Line 3
  e2 g2 e4 a4 a4 gis4 a2 r2 \break

  % Line 4
  a2 e2 a4 a4 g4 g4 fis2 e4 \break

  % Line 5
  a2 b4 c2 b2 r2 \break

  % Line 6
  b2 d2 c4( b2) a2 gis4 a1 \bar "|."
}
