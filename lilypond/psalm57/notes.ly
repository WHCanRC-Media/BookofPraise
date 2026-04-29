melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 c2 b2 a2 d2 c4 a4 b4 b4 a2 r2 \break

  % Line 2
  a2 b4 c4 d2 d2 c4 a4 b4 b4 a2 r2 \break

  % Line 3
  a2 c4 c4 b2 a2 g4 b4 a4 g4 fis2 e4 \break

  % Line 4
  g4 a4 b4 c2 c2 b4 a4 a4 gis4 a2 r2 \break

  % Line 5
  a2 c4 b4 a2 e2 fis4 a4 g4 fis4 e2 d1 \bar "|."
}
