melody = \relative c'' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 e2 a2 g4 g4 c4 c4 b2 a2 g2 r2 \break

  % Line 2
  d2 g4 a4 b2 b2 a4 g4 e2 e2 fis2 r2 \break

  % Line 3
  g2 a4 b4 e,4 b'4 c4 b4 a2 a2 g2 r2 \break

  % Line 4
  b2 a4 g4 fis2 e2 r2 \break

  % Line 5
  b2 b4 b4 a2 d,2 g4 a4 b4 c4 b2 a2 r2 \break

  % Line 6
  b2 b4 b4 e,2 fis2 g4 a4 b4 a4 g2 fis2 r2 \break

  % Line 7
  d2 e4 fis4 g4 g4 a4 b4 c4 b4 a2 g2 r2 \break

  % Line 8
  b2 a2 fis2 e1 \bar "|."
}
