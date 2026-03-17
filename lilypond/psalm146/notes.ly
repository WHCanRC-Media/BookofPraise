melody = \relative c'' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 b2 c2 b4 g4 a4 g4 fis2 e2 r2 \break

  % Line 2
  e2 g2 fis4 e4 e4 dis4 e2 r2 \break

  % Line 3
  g2 e2 g4 a4 b4 g4 c2 b2 r2 \break

  % Line 4
  b2 c2 b4 a4 a4 gis4 a2 r2 \break

  % Line 5
  a2 b2 g4 e4 g4 fis4 e2 r2 \break

  % Line 6
  e2 g2 e4 a4 a4 gis4 a1 \bar "|."
}
